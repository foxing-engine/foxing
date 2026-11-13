"""S6: a load test that is a Watch. N passing names on the live world at a player's
pace: keyboard players on a session (three subscriptions, as the play page holds),
agents through the gate and the plain command door. Reports what each reply cost.
usage: python3 load.py <bots> <minutes> [tag]"""
import asyncio, json, random, secrets, statistics, sys, time, urllib.request, urllib.error
import websockets
N=int(sys.argv[1]); MINUTES=float(sys.argv[2]); TAG=sys.argv[3] if len(sys.argv)>3 else "a"
B="https://play.agentscanplay.com/v0/"; WS="wss://play.agentscanplay.com/v0/session"
P="load-"+secrets.token_hex(8); T_END=time.time()+MINUTES*60
rows=[]      # (t, bot, door, verb, ok, client_ms, world_ms, wait_ms, log_ms, http)
push={"frames":0,"bytes":0}; errors=[]; probe=[]
def http(path, body=None, timeout=45):
    t=time.time()
    try:
        req=urllib.request.Request(B+path, json.dumps(body).encode() if body is not None else None, {"content-type":"application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as r: return r.status, json.loads(r.read()), (time.time()-t)*1000
    except urllib.error.HTTPError as e:
        try: j=json.loads(e.read())
        except Exception: j={}
        return e.code, j, (time.time()-t)*1000
    except Exception as e:
        return 0, {"error": type(e).__name__+": "+str(e)[:120]}, (time.time()-t)*1000
def rec(bot, door, verb, ok, ms, took, code):
    took=took or {}
    rows.append((time.time(), bot, door, verb, ok, ms, took.get("world_ms"), took.get("wait_ms"), took.get("log_ms"), code))
WORDS="lamp fog stair ledger moth kettle clock door river ink bell coat harbor stone letter window".split()
def story(): 
    w=random.sample(WORDS,8); return f"A {w[0]} stood by the {w[1]} while the {w[2]} waited. Nobody came for the {w[3]}, so the {w[4]} kept the {w[5]} and the {w[6]} forgot the {w[7]}."
ROUTE=["north","north","names","post","out","out","south","lane","row","south","east","west","west","east"]
def script(role, i, state):
    """The next command of a bot, by its role and what the last reply said."""
    last=state.get("last","")
    if role=="fighter":
        if not state.get("at"): state["at"]=True; return ("travel",[random.choice(["east","north"])])
        if "nothing here to fight" in last.lower() or "is gone" in last: 
            state["moves"]=state.get("moves",0)+1
            return random.choice([("look",[]),("rest",[]),("chunks",[]),("travel",[random.choice(["fogbank","lantern row","orchard"])])])
        if "You wake at the Threshold" in last or "You go down" in last: state["at"]=False; return ("mine",[])
        if "could be your last" in last: return ("retreat",[])
        return random.choices([("attack",[]),("look",[]),("rest",[])],[8,1,1])[0]
    if role=="walker":
        k=state.get("k",0); state["k"]=k+1
        if k%3==2: return random.choice([("look",[]),("map",[]),("time",[]),("wants",[]),("board",[]),("news",[]),("satchel",[]),("where",["docent"])])
        s=state.get("step",0); state["step"]=s+1; return ("travel",[ROUTE[s%len(ROUTE)]])
    # teller
    k=state.get("k",0); state["k"]=k+1
    if k==0: return ("travel",["east"])
    if k==2: return ("story",["Fallis", story()])
    return random.choice([("say",[random.choice(["quiet tonight","is anyone keeping count","the fog is thin","mind the Shallows"])]),("wants",[]),("look",[]),("tell",["Fallis","What do you want?"]),("messages",[]),("chunks",[])])
async def think(): await asyncio.sleep(random.uniform(2.5,6.5))
async def keyboard(i, role):
    name=f"Gnload{TAG}{i:02d}"
    code,j,ms=await asyncio.to_thread(http,"login",{"actor":name,"passphrase":P,"passing":True}); rec(name,"http","login",code==200,ms,j.get("took"),code)
    if code!=200: errors.append((name,"login",code,str(j)[:160])); return
    state={}; pending={}
    try:
        async with websockets.connect(WS, open_timeout=30, max_size=None) as ws:
            await ws.send(json.dumps({"type":"login","actor":name,"passphrase":P}))
            async def reader():
                async for m in ws:
                    f=json.loads(m)
                    if f.get("type")=="result" and f.get("id") in pending: pending.pop(f["id"]).set_result(f)
                    elif f.get("type") in ("event","observation"): push["frames"]+=1; push["bytes"]+=len(m)
            r=asyncio.create_task(reader())
            await asyncio.sleep(0.5)
            for sid,q in (("room",{"root":"here","select":{"disremembered.room":["name","description"]},"relations":{"contents":{"kinds":["citizen","creature","item","docent"],"select":{"disremembered.citizen":["name","passing"],"disremembered.creature":["name","hp","max_hp"]}},"exits":{"select":{"disremembered.room":["name"]}}},"timed":"curve"}),
                          ("me",{"root":"self","select":{"disremembered.body":["hp","max_hp"],"disremembered.citizen":["name","passing"]},"timed":"curve"}),
                          ("window",{"root":"window","select":{},"relations":{"contents":{"kinds":["chunk"],"select":{"disremembered.chunk":"*"}}},"timed":"curve"})):
                await ws.send(json.dumps({"type":"subscribe","id":sid,"query":q}))
            n=0
            while time.time()<T_END:
                await think(); verb,args=script(role,i,state); n+=1; cid=f"c{n}"
                fut=asyncio.get_event_loop().create_future(); pending[cid]=fut; t=time.time()
                await ws.send(json.dumps({"type":"command","id":cid,"command":verb,"args":args}))
                try: f=await asyncio.wait_for(fut,45)
                except asyncio.TimeoutError: rec(name,"ws",verb,False,45000,None,0); errors.append((name,verb,"timeout","")); continue
                state["last"]=f.get("summary") or f.get("error") or ""
                rec(name,"ws",verb,bool(f.get("ok")),(time.time()-t)*1000,f.get("took"),200)
            fut=asyncio.get_event_loop().create_future(); pending["bye"]=fut
            await ws.send(json.dumps({"type":"command","id":"bye","command":"leave","args":[]}))
            try: await asyncio.wait_for(fut,20)
            except Exception: pass
            r.cancel()
    except Exception as e:
        errors.append((name,"session",type(e).__name__,str(e)[:160]))
        await asyncio.to_thread(http,"command",{"actor":name,"passphrase":P,"command":"leave","args":[]})
async def agent(i, role, door):
    name=f"Gnload{TAG}{i:02d}"; state={}; n=0
    code,j,ms=await asyncio.to_thread(http,"login",{"actor":name,"passphrase":P,"passing":True}); rec(name,"http","login",code==200,ms,j.get("took"),code)
    if code!=200: errors.append((name,"login",code,str(j)[:160])); return
    while time.time()<T_END:
        await think(); await asyncio.sleep(random.uniform(1,4))   # an agent thinks longer
        if door=="gate":
            cmds=[script(role,i,state) for _ in range(random.randint(1,3))]; n+=1
            def lit(a): return str(a) if not isinstance(a,str) or (a.replace("-","").isalnum() and " " not in a) else '"'+a.replace('"',"'")+'"'
            msg=f"FROM: {name}\nTO: disremembered\nINTENT: REQUEST\nREF: {name}-{n}\n\n"+"\n".join(f"[DO] {name} MUST {v}{''.join(' '+lit(a) for a in args)}." for v,args in cmds)+"\n"
            code,j,ms=await asyncio.to_thread(http,"gate",{"actor":name,"passphrase":P,"message":msg})
            state["last"]=j.get("reply") or j.get("error") or ""
            rec(name,"gate","+".join(v for v,_ in cmds),code==200 and j.get("verdict")!="FAIL",ms,j.get("took"),code)
            if code!=200: errors.append((name,"gate",code,str(j)[:160]))
        else:
            verb,args=script(role,i,state)
            code,j,ms=await asyncio.to_thread(http,"command",{"actor":name,"passphrase":P,"command":verb,"args":args})
            state["last"]=j.get("summary") or j.get("error") or ""
            rec(name,"http",verb,code==200,ms,j.get("took"),code)
            if code not in (200,400,409,422): errors.append((name,verb,code,str(j)[:160]))
    await asyncio.to_thread(http,"command",{"actor":name,"passphrase":P,"command":"leave","args":[]})
async def prober():
    """One steady question through the run: the same cheap read, every three seconds."""
    while time.time()<T_END+2:
        code,j,ms=await asyncio.to_thread(http,"board"); probe.append((time.time(),"board",code,ms))
        t=time.time(); 
        try:
            with urllib.request.urlopen("https://play.agentscanplay.com/health",timeout=20) as r: h=json.loads(r.read()); probe.append((time.time(),"health",200,(time.time()-t)*1000,h.get("events")))
        except Exception as e: probe.append((time.time(),"health",0,(time.time()-t)*1000,None))
        await asyncio.sleep(3)
def pct(v,p): 
    v=sorted(v); return v[min(len(v)-1,int(len(v)*p))] if v else float("nan")
async def main():
    roles=["fighter","walker","teller"]; tasks=[asyncio.create_task(prober())]; t0=time.time()
    for i in range(N):
        role=roles[i%3]
        kind="ws" if i%5<3 else ("gate" if i%5==3 else "http")     # 60% keyboard, 20% gate, 20% plain
        tasks.append(asyncio.create_task(keyboard(i,role) if kind=="ws" else agent(i,role,kind)))
        await asyncio.sleep(random.uniform(0.5,2.0))               # they do not all arrive at once
    await asyncio.gather(*tasks, return_exceptions=True)
    dur=time.time()-t0
    json.dump({"rows":rows,"probe":probe,"errors":errors,"push":push,"n":N,"minutes":MINUTES},open(f"run-{TAG}.json","w"))
    cmd=[r for r in rows if r[3]!="login"]
    print(f"{N} names, {dur/60:.1f} min: {len(cmd)} commands ({len(cmd)/dur:.2f}/s), {sum(1 for r in cmd if not r[4])} refused or failed, {len(errors)} errors")
    for door in ("ws","http","gate"):
        c=[r[5] for r in cmd if r[2]==door]; w=[r[6] for r in cmd if r[2]==door and r[6] is not None]; q=[r[7] for r in cmd if r[2]==door and r[7] is not None]
        if c: print(f"  {door:4} n={len(c):4}  round trip p50 {pct(c,.5):6.0f}  p95 {pct(c,.95):6.0f}  max {max(c):6.0f} ms | world p50 {pct(w,.5):5.1f} p95 {pct(w,.95):5.1f} max {max(w) if w else float('nan'):6.1f} | lock wait p95 {pct(q,.95):5.1f} max {max(q) if q else float('nan'):6.1f}")
    byv={}
    for r in cmd:
        if r[6] is not None and r[2]!="gate": byv.setdefault(r[3],[]).append(r[6])
    print("  world ms by verb (p50 / max, n):", "; ".join(f"{v} {pct(x,.5):.1f}/{max(x):.0f} ({len(x)})" for v,x in sorted(byv.items(), key=lambda kv:-max(kv[1]))[:12]))
    b=[p[3] for p in probe if p[1]=="board"]; h=[p[3] for p in probe if p[1]=="health"]
    print(f"  probe: board p50 {pct(b,.5):.0f} p95 {pct(b,.95):.0f} max {max(b):.0f} ms; health p50 {pct(h,.5):.0f} p95 {pct(h,.95):.0f} max {max(h):.0f} ms; non-200: {sum(1 for p in probe if p[2]!=200)}")
    ev=[p[4] for p in probe if p[1]=="health" and len(p)>4 and p[4]]
    if len(ev)>1: print(f"  events written: {ev[-1]-ev[0]} ({(ev[-1]-ev[0])/dur:.1f}/s); pushes to {sum(1 for i in range(N) if i%5<3)} sessions: {push['frames']} frames, {push['bytes']/1e6:.1f} MB ({push['bytes']/max(1,push['frames'])/1000:.1f} kB each)")
    codes={}
    for r in rows: codes[r[9]]=codes.get(r[9],0)+1
    print("  http codes:", codes); 
    for e in errors[:8]: print("  error:", e)
asyncio.run(main())

# Security

Foxing is alpha. One world runs on it in public (Disremembered), and holds what its players give it: names, passphrase hashes, and a public record of what they did. The design has two security boundaries that matter from the first commit: the module sandbox (a module gets nothing it did not declare) and the CLEAR gate (no uninspectable channel crosses it).

If you find a way for a module to reach state, commands, events or clocks it did not declare, for a query to return a field its actor may not see, or for a CLEAR message to reach the driver without passing the checker, please report it privately to hello@foxing.dev rather than opening an issue. Include a minimal reproduction. You will get an acknowledgement within a few days and credit in the fix unless you prefer otherwise.

Reports about the websites (agentscanplay.com, foxing.dev) go to the same address.

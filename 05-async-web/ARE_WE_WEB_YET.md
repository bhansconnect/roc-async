No, definitely not yet, but I decided to write up some basic tests anyway.

The following results all use async rust + hyper + tokio as a base for async web.
The Roc version does everything from routing to generating the response body and status code.

The tests routes include:
 - `/`: `Hello, World!` response.
 - `/hello/<first?>/<last?>`: Dynamic response saying `Hello, <first> <last>!`. Also has variants for missing names.
 - `/compute/<n>`: Computes the <n>th fibonacci number the bad recursive way to waste CPU cycles.
 - `/sleep/<ms>/<n?>`: Made to test async works correctly. Sleeps for <ms> time and repeat <n> times (<n> times not yet in Roc due to bug).
 - `/template`: Returns the roc-lang.org home page. Generates the list of talks from a list of {name, url, description}.
 - `/dup/<n>`: Is a POST route. Duplicates the body n times and returns that. Has to async wait for body to load.

Ran some performance tests on a minisforum z83-f mini pc (4 core intel atom, 4 GB ram, 20GB swap on SSD).
Used a powerful desktop as the test driver and network bandwidth was never a limiting factor.
The abitrary test criteria was finding the max RPS that can be substained for a minute while having a 99.9% latency below 300ms.
> Note: Benchmarking was done using [wrk2](https://github.com/giltene/wrk2). `wrk2` was configured with 1000 concurrent connections. The exception is `/compute` which uses 10 concurrent connections to avoid overloading the server.

| Endpoint | Roc RPS | Rust RPS | Ratio | Notes |
| --- | ---: | ---: | ---: | --- |
| `/` | 38,000 | 40,000 | 0.95 |  |
| `/hello/Jane/Doe` | 37,000 | 40,000 | 0.93 |  |
| `/compute/30` | 150 | 260 | 0.58 | ~25 ms of CPU work per request in Roc; ~15ms Rust |
| `/sleep/25` | 29,000 | 29,000 | 1.00 | We can do nothing as fast as Rust... Yay! |
| `/template` | 23,000 | 30,000 | 0.77 | Roc output has less whitespace due to how the templating was done |
| `/dup/20` | 32,000 | 35,000 | 0.91 | Send 50 bytes and duplicated to 1KB |


That being said, I think the biggest pieces that would be need to make Roc even vaguely server ready would be:
 - DB query creation and mapping to host (huge super important piece). How do we make it nicely typed? Does it have to be coordinated with the host? Does Roc have to generate the SQL query string for this to be generic? How can prepared statements be used (for speed and sql injection defence)? etc
 - Good library to define web page or templates (like elm?). I made an untyped form that actually turned out ok. Would be great to make a form that runs once to generate a template and from that point on is just doing a minimal transformation.
 - Good http request library for communicating with other services (in progress by other people).
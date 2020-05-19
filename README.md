# Ruply
A clojure [nREPL] command line client for quick clojure repl sessions.
Ruply is written in Rust so it compiles down to single compact binary and no java/clojure stuff is needed for running it.

It works also with pREPL (socket repl included in clojure v1.9->).  

Ruply detects automatically whether the server is nREPL or pREPL and just works.

## Building with cargo

```
% cargo build --release
```

Binary is now in the cargo target directory.


## Running
With nrepl-server running at localhost port 38581:

```
% ruply -h localhost -p 38581 

Connected to nREPL at localhost:38581
Exit: CTRL+D

user=> 
```

[nREPL]: https://nrepl.org/

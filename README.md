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


## Usage
With nrepl-server running at port 38581 and prepl-server at 5555.

### Running a repl session

```
% ruply -h localhost -p 38581 

Connected to nREPL at localhost:38581
Exit: CTRL+D

user=> (+ 1 1)
2
user=>
CTRL-D
```

### Executing single-shot snippets

```
 $ ruply -p 5555 -e "(last (sort (ns-map 'user)))"
[zipmap #'clojure.core/zipmap]

 $ ruply -p 5555 -e '(apply str (reverse "Hello"))'
"olleH"

```

[nREPL]: https://nrepl.org/

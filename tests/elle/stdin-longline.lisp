(elle/epoch 12)
## tests/elle/stdin-longline.lisp
##
## A line on STDIN longer than the buffer `port/read-line` reserves is
## answered without loss, and the read after it still frames the stream
## correctly. See docs/io.md § "A read that overshoots keeps the rest for
## the same port".
##
## tests/elle/port-longline.lisp asserts the same property for a socket,
## and that one passes: a socket read completes through
## `pool_to_completion`, which hands the worker's bytes to
## `complete_port_op` and lets `read_result` answer from the requesting
## instance's heap when they outgrow the reservation.
##
## Stdin does not take that path. It has its own worker
## (`src/io/threadpool/stdin.rs`) and its own converter
## (`stdin_to_completion`), and that converter kept the older shape: it
## copied the worker's bytes into the fiber's pre-allocated buffer under
## `data.len().min(dst_cap)`. `read_line_with_cancel` reads to the newline
## however far away it is, so everything past 64 KiB was dropped — bytes
## already out of the kernel, with nothing left to read them again. The
## fiber got a truncated line and no way to tell it was truncated.
##
## The counter-factual: a payload under 64 KiB passes every assertion here
## on the unfixed code. The line has to outgrow the reservation before the
## defect is reachable, which is why the payload is 200 KiB.
##
## We test through a subprocess so `make test` can run this without piping
## stdin into the runner itself.

(def line-size 200000)

(def long-line
  (let [@buf @""
        @i 0]
    (while (< i line-size)
      (push buf (string (mod i 10)))
      (assign i (+ i 1)))
    (freeze buf)))

## The child rebuilds the same digit pattern and compares, so a read that
## answers with the right LENGTH but the wrong bytes still fails.
(def inner-script
  "(def line-size 200000)
   (def expected
     (let [@buf @\"\"
           @i 0]
       (while (< i line-size)
         (push buf (string (mod i 10)))
         (assign i (+ i 1)))
       (freeze buf)))
   (def first-line (port/read-line (*stdin*)))
   (def second-line (port/read-line (*stdin*)))
   (println (length first-line))
   (println (if (= first-line expected) \"same\" \"differs\"))
   (println second-line)
   (sys/exit 0)")

(def elle-bin
  (cond
    (file/exists? "./target/release/elle") "./target/release/elle"
    (file/exists? "./target/debug/elle") "./target/debug/elle"
    true (error {:error :test-skip
                 :message "cannot find elle binary in ./target/"})))

(def scratch (file/mktempdir))
(def inner-path (path/join scratch "stdin-longline-inner.lisp"))
(def input-path (path/join scratch "stdin-longline-input.txt"))
(file/write inner-path inner-script)

## The long line, then a short one. The second line is what proves the
## overshoot was framed rather than merely delivered: a converter that
## mislays the remainder loses it or replays part of the first line here.
(file/write input-path (concat long-line "\ntail\n"))

(def result
  (subprocess/system "sh"
                     ["-c"
                      (string "cat '" input-path "' | '" elle-bin
                              "' '" inner-path "'")]))

(assert (= result:exit 0)
        (string "subprocess exited " result:exit ": " result:stderr))

(def lines (string/split (string/trim result:stdout) "\n"))

(assert (= (get lines 0) (string line-size))
        (string "the whole line is answered: got " (get lines 0)
                " of " (string line-size)))
(println "  1. a stdin line past its buffer is answered whole")

(assert (= (get lines 1) "same")
        "and byte for byte, not merely the right length")
(println "  2. and byte for byte")

(assert (= (get lines 2) "tail")
        (string "the next read resumes after the newline: got "
                (get lines 2)))
(println "  3. the read after it frames the stream correctly")

(println "stdin-longline: ok")

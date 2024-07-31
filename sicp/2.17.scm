(def last-pair
  (fn (xs)
    (cond
      ((not (null? (cdr xs))) (last-pair (cdr xs)))
      (true xs))))

(assert
  (last-pair '(23 72 149 34))
  (list 34))

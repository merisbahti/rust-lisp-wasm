; louis's code
(flatmap
  (lambda (new-row)
    (map (lambda (rest-of-queens)
          (adjoin-position new-row k rest-of-queens))
      (queen-cols (- k 1))))
  (enumerate-interval 1 board-size))

; original code
(flatmap
  (lambda (rest-of-queens)
    (map (lambda (new-row)
          (adjoin-position new-row k rest-of-queens))
      (enumerate-interval 1 board-size)))
  (queen-cols (- k 1)))

; in the original code, if we call queens-cols 5, we'll recursively call it 4 more times.
; in louis's code, if we call queen 5, we'll call (queen-cols 4) 3 times (instead of just 1 time)

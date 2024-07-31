(define (fringe xs)
  (cond
    ((null? xs) xs)
    ((pair? xs) (append
                 (fringe (car xs))
                 (fringe (cdr xs))))
    (else (list xs))))
(define x (list (list 1 2) (list 3 4)))
(assert (fringe x) '(1 2 3 4))
(assert
  (fringe (list x x))
  '(1 2 3 4 1 2 3 4))

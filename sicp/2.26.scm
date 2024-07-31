(define x (list 1 2 3))
(define y (list 4 5 6))

(assert (append x y) '(1 2 3 4 5 6))

(assert (cons x y) (list x 4 5 6))

(assert (list x y) (list x y))

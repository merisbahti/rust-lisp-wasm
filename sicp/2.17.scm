(define a 1)
(define (f x) ; lambda closing over a
  (define b 1)
  (define (g y) ; lambda closing over a, b, x
    (+ a b x y)))

(define last-pair
  (lambda (xs)
    (cond
      ((not (null? (cdr xs))) (last-pair (cdr xs)))
      (true xs))))

(assert
  (last-pair '(23 72 149 34))
  (list 34))

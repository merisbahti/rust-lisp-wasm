(define (odd? x) (= (% x 2) 1))
(define (even? x) (= (% x 2) 0))
(define (f y) (odd? y))
(define parityyy 1)
(define
  (same-parity parity . xs)
  (let
    (
      (parityFn (cond
                 ((odd? parity) odd?)
                 (else even?))))
    (cons parity (filter parityFn xs))))

(assert (same-parity 1 2 3 4 5 6 7) (list 1 3 5 7))

(assert (same-parity 2 3 4 5 6 7) (list 2 4 6))

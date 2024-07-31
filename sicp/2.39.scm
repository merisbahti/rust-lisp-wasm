(define (reverse-fold-r sequence)
  (fold-right
    (lambda (curr acc)
      (append acc (list curr)))
    nil
    sequence))

(assert (reverse-fold-r (list 1 2 3)) (list 3 2 1))

(define (reverse-fold-l sequence)
  (fold-left
    (lambda (acc curr) (cons curr acc))
    nil
    sequence))

(assert (reverse-fold-l (list 1 2 3)) (list 3 2 1))

(assert
  (list 1 2 3 4)
  (cons 1 (cons 2 (cons 3 (cons 4 null)))))

(assert (list 3 4) (cons 3 (cons 4 null)))

(assert (list 2 (list 3 4)) (cons 2 (cons (cons 3 (cons 4 null)) null)))

(assert
  (list 1 (list 2 (list 3 4)))
  (cons 1 (cons (cons 2 (cons (cons 3 (cons 4 null)) null)) null)))

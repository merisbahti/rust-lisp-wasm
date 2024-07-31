(define (unique-triples n)
  (flatmap
    (lambda (i) (flatmap
                 (lambda (j) (map
                              (lambda (k) (list k j i))
                              (enumerate-interval 1 (- j 1))))
                 (enumerate-interval 1 (- i 1))))
    (enumerate-interval 1 n)))

(assert (unique-triples 5) (list
                            (list 1 2 3)
                            (list 1 2 4)
                            (list 1 3 4)
                            (list 2 3 4)
                            (list 1 2 5)
                            (list 1 3 5)
                            (list 2 3 5)
                            (list 1 4 5)
                            (list 2 4 5)
                            (list 3 4 5)))
;

;  Write a procedure to find all ordered triples of distinct positive integers i, j, and k less than or equal to a given integer n that sum to a given integer s.

(define (exproc n s)
  (filter
    (lambda (triple) (= n (apply + triple)))
    (unique-triples s)))

(assert
  (exproc 8 8)
  (list (list 1 3 4) (list 1 2 5)))

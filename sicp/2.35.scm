(define (count-leaves t)
  (accumulate
    (lambda (curr acc) (+ curr acc))
    0
    (map
      (lambda (sub-tree-maybe)
        (cond
          ((pair? sub-tree-maybe) (count-leaves sub-tree-maybe))
          ((number? sub-tree-maybe) 1)))
      t)))

(define x (cons (list 1 2) (list 3 4)))

(assert (count-leaves x) 4)
(define x2 (cons (list 1
                  (list 1 2)
                  2)
            (list 3 4)))

(assert (count-leaves x2) 6)

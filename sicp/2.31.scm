;; Exercise 2.31.  Abstract your answer to exercise 2.30 to produce a procedure tree-map with the property that square-tree could be defined as

;; Define a procedure square-tree analogous to the square-list procedure of exercise 2.21.
;; That is, square-list should behave as follows:

(define (square x) (* x x))
(define (tree-map f tree)
  (map
    (lambda (sub-tree)
      (if
        (pair? sub-tree)
        (tree-map f sub-tree)
        (f sub-tree)))
    tree))
(define (square-tree tree) (tree-map square tree))

(assert
  (square-tree
    (list 1
      (list 2 (list 3 4) 5)
      (list 6 7)))
  '(1 (4 (9 16) 25) (36 49)))

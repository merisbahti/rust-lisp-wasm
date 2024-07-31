;; Define a procedure square-tree analogous to the square-list procedure of exercise 2.21.
;; That is, square-list should behave as follows:
(define (square-tree tree)
  (cond
    (
      (null? tree)
      tree)
    (
      (number? tree)
      (* tree tree))
    (
      (pair? tree)
      (cons (square-tree (car tree)) (square-tree (cdr tree))))))

(define (square-tree-map tree)
  (map
    (fn (sub-tree)
      (if
        (pair? sub-tree)
        (square-tree-map sub-tree)
        (* sub-tree sub-tree)))
    tree))

(assert
  (square-tree
    (list 1
      (list 2 (list 3 4) 5)
      (list 6 7)))
  '(1 (4 (9 16) 25) (36 49)))

(assert
  (square-tree-map
    (list 1
      (list 2 (list 3 4) 5)
      (list 6 7)))
  '(1 (4 (9 16) 25) (36 49)))

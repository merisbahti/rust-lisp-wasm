(define (square x) (* x x))
(define (square-list1 items)
  (cond
    ((null? items) '())
    (else
      (cons (square (car items)) (square-list1 (cdr items))))))

(define (square-list2 items)
  (map square items))

(assert
  (square-list1 (list 1 2 3 4))
  (list 1 4 9 16))
(assert
  (square-list2 (list 1 2 3 4))
  (list 1 4 9 16))

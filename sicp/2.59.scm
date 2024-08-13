(define (element-of-set? x set)
  (cond ((null? set) false)
    ((= x (car set)) true)
    (else (element-of-set? x (cdr set)))))

(define (adjoin-set x set)
  (if (element-of-set? x set)
    set
    (cons x set)))

(define (intersection-set set1 set2)
  (cond
    ((or (null? set1) (null? set2)) '())
    ((element-of-set? (car set1) set2)
      (cons (car set1)
        (intersection-set (cdr set1) set2)))
    (else (intersection-set (cdr set1) set2))))

(assert (element-of-set? 1 '(1 2 3)) true)
(assert (element-of-set? 4 '(1 2 3)) false)
(assert (adjoin-set 1 '(1 2 3)) '(1 2 3))
(assert (adjoin-set 4 '(1 2 3)) '(4 1 2 3))
(assert (intersection-set '(3 4) '(1 2 3)) '(3))

(define (union-set set1 set2)
  (cond
    ((null? set2) set1)
    ((null? set1) set2)
    ((element-of-set? (car set1) set2)
      (union-set (cdr set1) set2))
    (else
      (cons (car set1)
        (union-set (cdr set1) set2)))))

(assert (union-set '(3 4) '(1 2 3)) '(4 1 2 3))

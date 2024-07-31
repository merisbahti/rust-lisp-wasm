(define (fold-left op initial sequence)
  (define (iter result rest)
    (if (null? rest)
      result
      (iter (op result (car rest))
        (cdr rest))))
  (iter initial sequence))

(define (fold-right op initial sequence)
  (cond
    ((nil? sequence)
      initial)
    (else
      (op
        (car sequence)
        (fold-right op initial (cdr sequence))))))

(assert (fold-left + 0 (list 1 2 3)) 6)
(assert (fold-left / 1 (list 1 2 3)) (/ 0.5 3))
; (/ (/ (/ 1 1) 2) 3)
; (/ (/ 1 2) 3)
; (/ 0.5 3)
; 0.1666...

(assert (accumulate / 1 (list 1 2 3)) 1.5)

; (/ 1 (/ 2 (/ 3 1)))
; (/ 1 (/ 2 3))
; (/ 1 (/ 2 3))
; (/ 1 0.666)
; 1.5

; Answer: The operator should be commutative (x * y) = (y * x)

(define (unique-pairs n)
  (flatmap
    (lambda (i) (map
                 (lambda (j) (list j i))
                 (enumerate-interval 1 (- i 1))))
    (enumerate-interval 1 n)))

(assert (unique-pairs 3) (list (list 1 2) (list 1 3) (list 2 3)))

(define (make-pair-sum pair)
  (define fst (car pair))
  (define snd (car (cdr pair)))
  (list fst snd (+ fst snd)))

(define (smallest-divisor n)
  (find-divisor n 2))

(define (square n) (* n n))
(define (divides? test-divisor n) (= (% n test-divisor) 0))

(define (find-divisor n test-divisor)
  (cond ((> (square test-divisor) n) n)
    ((divides? test-divisor n) test-divisor)
    (else (find-divisor n (+ test-divisor 1)))))

(define (prime-sum? n)
  (define fst (car n))
  (define snd (car (cdr n)))
  (prime? (+ fst snd)))
(define (prime? n) (= n (smallest-divisor n)))

(define (prime-sum-pairs n)
  (map make-pair-sum
    (filter prime-sum?
      (unique-pairs n))))

; (1 2 3)
; 1.(2,3)
; (list 1 2 3)
; list(1,2,3)
(assert
  (prime-sum-pairs 6)
  (list
    (list 1 2 3)
    (list 2 3 5)
    (list 1 4 5)
    (list 3 4 7)
    (list 2 5 7)
    (list 1 6 7)
    (list 5 6 11)))

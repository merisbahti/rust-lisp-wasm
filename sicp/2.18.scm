(define (reverse x)
  (define (reverse-iter x acc)
    (cond
      ((null? x) acc)
      (else (reverse-iter (cdr x) (cons (car x) acc)))))
  (reverse-iter x '()))

(assert
  (reverse '(1 4 9 16 25))
  (list 25 16 9 4 1))

(define (deep-reverse xs)
  (cond
    ((null? xs) xs)
    ((not (pair? xs)) xs)
    ((pair? (car xs)) ())
    (else
      (append
        (deep-reverse (cdr xs))
        (deep-reverse (car xs))))))

(define (deep-reverse x)
  (define (deep-reverse-iter x acc)
    (cond
      ((null? x) acc)
      ((pair? (car x))
        ((lambda ()
            (define reversed (deep-reverse (car x)))
            (deep-reverse-iter (cdr x) (cons reversed acc)))))
      (else (deep-reverse-iter (cdr x) (cons (car x) acc)))))
  (deep-reverse-iter x '()))

(define x (list (list 1 2) (list 3 4)))
(assert (deep-reverse x) (list (list 4 3) (list 2 1)))

(define x2 (list (list 1 2) (list 3 4 (list 5 6))))
(assert (deep-reverse x2) (list (list (list 6 5) 4 3) (list 2 1)))

;; To be more precise, we can define equal? recursively in terms of the basic eq? equality of symbols by saying that a and b are equal?
;; if they are both symbols and the symbols are eq?, or if they are both lists such that (car a) is equal? to (car b) and (cdr a) is equal? to (cdr b).
;; Using this idea, implement equal? as a procedure.  (define (equal2? something)
(define (equal2? x y)
  (cond
    ((and (null? x) (null? y)) true)
    ((and (symbol? x) (and (symbol? y) (= x y))) true)
    ((and (pair? x) (and (pair? y) (and (not (null? x)) (not (null? y)))))
      (and
        (equal2? (car x) (car y))
        (equal2? (cdr x) (cdr y))))
    (else false)))

(assert
  (equal2? '(this is a list) '(this is a list))
  true)

(assert (equal2? '(this is a list) '(this (is a) list)) false)

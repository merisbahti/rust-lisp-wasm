(define x1 '(1 3 (5 7) 9))
(assert (car (cdr (car (cdr (cdr x1))))) 7)

(define x2 '((7)))
(assert (car (car x2)) 7)

(define x3 '(1 (2 (3 (4 (5 (6 7)))))))
(assert (car (cdr (car (cdr (car (cdr (car (cdr (car (cdr (car (cdr x3)))))))))))) 7)

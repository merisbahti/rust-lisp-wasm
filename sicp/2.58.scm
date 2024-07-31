(define (deriv exp var)
  (cond ((number? exp) 0)
    ((variable? exp)
      (if (same-variable? exp var) 1 0))
    ((sum? exp)
      (make-sum (deriv (addend exp) var)
        (deriv (augend exp) var)))
    ((product? exp)
      (make-sum
        (make-product (multiplier exp)
          (deriv (multiplicand exp) var))
        (make-product (deriv (multiplier exp) var)
          (multiplicand exp))))
    (else
      (error "unknown expression type -- DERIV"))))
(define (variable? x) (symbol? x))

(define (same-variable? v1 v2)
  (and (variable? v1) (and (variable? v2) (= v1 v2))))

(assert (same-variable? 'x 'y) false)
(assert (same-variable? 'x 'x) true)

(define (=number? exp num)
  (and (number? exp) (= exp num)))

(define (make-sum a1 a2)
  (cond ((=number? a1 0) a2)
    ((=number? a2 0) a1)
    ((and (number? a1) (number? a2)) (+ a1 a2))
    (else (list a1 '+ a2))))

(define (sum? x)
  (and (pair? x) (= (car (cdr x)) '+)))
(define (addend s) (car s))
(define (augend s)
  (car (cdr (cdr s))))

(define (make-product m1 m2)
  (cond ((or (=number? m1 0) (=number? m2 0)) 0)
    ((=number? m1 1) m2)
    ((=number? m2 1) m1)
    ((and (number? m1) (number? m2)) (* m1 m2))
    (else (list m1 '* m2))))

(define (product? x)
  (and (pair? x) (= (car (cdr x)) '*)))
(define (multiplier p) (car p))
(define (multiplicand p)
  (car (cdr (cdr p))))

(assert
  (deriv '((x * y) * (x + 3)) 'x)
  '((x * y) + (y * (x + 3))))

(assert (deriv '(x * y) 'x) 'y)

(assert (deriv '(x + 3) 'x) '1)

(assert (deriv '(x * (y * (x + 3))) 'x) '((x * y) + (y * (x + 3))))

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
    ((exponent? exp)
      (make-product (exponent exp) (make-exponent (base exp) (make-sum (exponent exp) -1))))
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
    (else (list '+ a1 a2))))

(define (sum? x)
  (and (pair? x) (= (car x) '+)))
(define (addend s) (cadr s))
(define (augend s)
  (define rest (cdr (cdr s)))
  (accumulate (lambda (args acc) (make-sum args acc)) 0 rest))

(define (make-product m1 m2)
  (cond ((or (=number? m1 0) (=number? m2 0)) 0)
    ((=number? m1 1) m2)
    ((=number? m2 1) m1)
    ((and (number? m1) (number? m2)) (* m1 m2))
    (else (list '* m1 m2))))

(define (product? x)
  (and (pair? x) (= (car x) '*)))
(define (multiplier p) (cadr p))
(define (multiplicand p)
  (define rest (cdr (cdr p)))
  (define res (accumulate (lambda (args acc) (make-product args acc)) 1 rest))

  res)

(assert
  (deriv '(* (* x y) (+ x 3)) 'x)
  '(+ (* x y) (* y (+ x 3))))
(assert (deriv '(* x y) 'x) 'y)

(define (make-exponent base exponent)
  (cond ((=number? base 1) 1)
    ((=number? exponent 1) base)
    ((=number? exponent 0) 1)
    (else (list '** base exponent))))

(define (exponent? x)
  (and (pair? x) (= (car x) '**)))
(define (base p) (cadr p))
(define (exponent p) (caddr p))

(assert (deriv (make-exponent 'x 2) 'x) '(* 2 x))
(assert (deriv '(+ x 3) 'x) '1)
(assert (deriv (make-exponent 'x 3) 'x) '(* 3 (** x 2)))
(assert (deriv '(* x y (+ x 3)) 'x) '(+ (* x y) (* y (+ x 3))))

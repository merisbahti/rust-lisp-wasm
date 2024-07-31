(define (pow x exp)
  (define (pow-iter curr-iter acc)
    (cond
      ((= curr-iter 0) acc)
      ((< curr-iter 0) (pow-iter (+ curr-iter 1) (/ acc x)))
      ((> curr-iter 0) (pow-iter (- curr-iter 1) (* acc x)))))
  (pow-iter exp 1))

;; (assert (pow 2 3) 8)
(assert (< -3 0) true)
(assert (+ -1 1) 0)
(assert (pow 2 -3) (/ 1 8))

(define (horner-eval-meris x polynomial)
  (define (op factor acc)
    (define sum (car acc))
    (define index (cdr acc))
    (define next-acc (cons (+ sum (* factor (pow x index))) (- index 1)))
    next-acc)
  (car (accumulate op (cons 0 (- (length polynomial) 1)) polynomial)))

;; 1 + 3x^1 + 5x^3 + x^5 at x = 2
(assert
  (horner-eval-meris 2 (list 1 3 0 5 0 1))
  79)

(define (horner-eval x coefficient-sequence)
  (accumulate (lambda (this-coeff higher-terms) (+ (* higher-terms x) this-coeff))
    0
    coefficient-sequence))

(assert
  (horner-eval 2 (list 1 3 0 5 0 1))
  79)

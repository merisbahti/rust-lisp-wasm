(define (make-mobile left right)
  (list left right))

(define (make-branch length structure)
  (list length structure))

(define (left-branch mobile)
  (car mobile))
(define (right-branch mobile)
  (car (cdr mobile)))

(define (branch-length branch) (car branch))
(define (branch-structure branch) (car (cdr branch)))

(define example-mobile
  (make-mobile
    (make-branch 8 5)
    (make-branch 5 (make-mobile
                    (make-branch 2 1)
                    (make-branch 5 1)))))

(define (branch-weight branch)
  (define structure (branch-structure branch))
  (cond
    ((number? structure) structure)
    (else (total-weight structure))))

(define (total-weight mobile)
  (+
    (branch-weight (left-branch mobile))
    (branch-weight (right-branch mobile))))

(assert (total-weight example-mobile) 7)

;; c:
;; A mobile is said to be balanced if the torque applied by its top-left branch is equal to that applied by its top-right branch
;; (that is, if the length of the left rod multiplied by the weight hanging from that rod is equal to the corresponding product
;; for the right side) and if each of the submobiles hanging off its branches is balanced.
;; Design a predicate that tests whether a binary mobile is balanced.
(define (mobile-balanced? mobile)
  (and
    (=
      (branch-torque (left-branch mobile))
      (branch-torque (right-branch mobile)))
    (and
      (branch-balanced? (left-branch mobile))
      (branch-balanced? (right-branch mobile)))))

(define (branch-balanced? branch)
  (define structure (branch-structure branch))
  (cond
    ((number? structure) true)
    (else (mobile-balanced? structure))))

(define (branch-torque branch)
  (*
    (branch-length branch)
    (branch-weight branch)))

;; simple case
(define balanced-mobile
  (make-mobile
    (make-branch 10 10)
    (make-branch 10 10)))
(assert (mobile-balanced? balanced-mobile) true)

;; bit more complciated case
(define balanced-complex-mobile
  (make-mobile
    (make-branch 10 (make-mobile
                     (make-branch 5 20)
                     (make-branch 20 5)))
    (make-branch 10 25)))
(assert (mobile-balanced? balanced-complex-mobile) true)

(assert (mobile-balanced? example-mobile) false)

;; d: "change (car (cdr mobile/branch)) to (cdr mobile/branch)"

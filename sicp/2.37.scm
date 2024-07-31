(define (dot-product v w)
  (accumulate + 0 (map-n * v w)))
(define matrix '((1 2 3 4) (4 5 6 6) (6 7 8 9)))

(assert
  (map-n (lambda (a b) (+ a b)) (car matrix) (car (cdr matrix)))
  '(5 7 9 10))

(assert (dot-product '(1 2 3) '(4 -5 6)) 12)
(assert (dot-product (car matrix) (car (cdr matrix))) (+ (* 1 4) (* 2 5) (* 6 3) (* 6 4)))

(define
  (matrix-*-vector m v)
  (map (lambda (mi) (dot-product mi v)) m))

(assert (matrix-*-vector
         (list
           (list 1 2 3)
           (list 4 5 6)
           (list 7 8 9))
         (list 2 1 3))
  (list 13 31 49))

(assert (matrix-*-vector
         (list (list 1 -1 2) (list 0 -3 1))
         (list 2 1 0))
  (list 1 -3))

(define
  (transpose mat)
  (accumulate-n
    (lambda (curr acc) (cons curr acc))
    '()
    mat))

(assert
  (transpose
    (list
      (list 1 2 3)
      (list 4 5 6)
      (list 7 8 9)))
  (list
    (list 1 4 7)
    (list 2 5 8)
    (list 3 6 9)))

(define
  (matrix-*-matrix m n)
  (let ((cols (transpose n)))
    (map
      (lambda
        (m-row)
        (matrix-*-vector cols m-row))
      m)))

(assert
  (matrix-*-matrix
    (list
      (list 1 2 3)
      (list 4 5 6)
      (list 7 8 9))
    (list
      (list 1 2 3)
      (list 4 5 6)
      (list 7 8 9)))
  (list
    (list 30 36 42)
    (list 66 81 96)
    (list 102 126 150)))

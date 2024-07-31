(define (map p sequence)
  (accumulate (lambda (x y) (cons (p x) y)) nil sequence))

(assert
  (map (lambda (x) (+ 1 x)) '(1 10 15))
  '(2 11 16))

(assert
  (map (lambda (x) (+ x x)) '(1 10 15))
  '(2 20 30))

(define old-append (append '(1 2 3) '(4 5 6)))

(define (append seq1 seq2)
  (accumulate cons seq2 seq1))

(assert
  (append '(1 2 3) '(4 5 6))
  old-append)

(define (length sequence)
  (accumulate (lambda (skip x) (+ 1 x)) 0 sequence))

(assert
  (length '(1 2 3 4 5))
  5)

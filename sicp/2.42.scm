(define (queens board-size)
  (define (queen-cols k)
    (if (= k 0)
      (list empty-board)
      (filter
        (lambda (positions)
          (safe? k positions))
        (flatmap
          (lambda (rest-of-queens)
            (map (lambda (new-row)
                  (adjoin-position new-row k rest-of-queens))
              (enumerate-interval 1 board-size)))
          (queen-cols (- k 1))))))
  (queen-cols board-size))

(define
  (get-diagonals-backwards pos)
  (define row (car pos))
  (define col (car (cdr pos)))
  (flatmap
    (lambda (currCol)
      (list
        (list (- row currCol) (- col currCol))
        (list (+ row currCol) (- col currCol))))
    (enumerate-interval 0 col)))
(define (exists? pred coll)
  (cond
    ((null? coll) false)
    ((pred (car coll)) true)
    (else (exists? pred (cdr coll)))))

(define (on-diagonal pos new-queen)
  (exists? (lambda (diagonal-pos) (= diagonal-pos pos)) (get-diagonals-backwards new-queen)))

(assert (on-diagonal (list 4 4) (list 5 5)) true)
(assert (on-diagonal (list 4 3) (list 5 5)) false)
(assert (on-diagonal (list 1 1) (list 5 5)) true)

(define (safe? k all-positions)
  (define newQueen (car all-positions))
  (define positions (cdr all-positions))
  (define newQueenRow (car newQueen))
  (define newQueenCol (car (cdr newQueen)))
  (not
    (exists?
      (lambda (pos)
        (define posRow (car pos))
        (define posCol (car (cdr pos)))
        (or
          (= posCol newQueenCol)
          (or (= posRow newQueenRow)
            (on-diagonal pos newQueen))))
      positions)))

(define
  (adjoin-position row col rest-of-queens)
  (cons (list row col) rest-of-queens))

(define empty-board '())

(assert (length (queens 5)) 10)

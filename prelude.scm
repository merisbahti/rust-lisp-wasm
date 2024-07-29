(define else true)

(defmacro (syntax-list . xs)
  (define (fold-right op initial sequence)
    (if
      (nil? sequence)
      initial
      (op
        (car sequence)
        (fold-right op initial (cdr sequence)))))
  (fold-right
    (lambda (curr acc)
      (cons 'cons (cons curr (cons acc '()))))
    '()
    xs))

(define (list . xs) xs)

(define (progn . xs)
  (cons 'lambda (cons '() xs)))

(define (print . xs)
  (define (fold-right op initial sequence)
    (if
      (nil? sequence)
      initial
      (op
        (car sequence)
        (fold-right op initial (cdr sequence)))))
  (display
    (fold-right
      (lambda (curr acc)
        (str-append (to-string curr) acc))
      ""
      xs)))

(defmacro (cond . exprs)
  (define (fold-right op initial sequence)
    (if

      (nil? sequence)
      initial
      (op
        (car sequence)
        (fold-right op initial (cdr sequence)))))
  (fold-right (lambda (curr acc)
               (define predicate (car curr))
               (define consequent (car (cdr curr)))
               (cons 'if (cons predicate (cons consequent (cons acc '())))))
    '()
    exprs))

(defmacro (dprint . exprs)
  (define (map proc items)
    (cond
      ((nil? items) '())
      (true
        (cons (proc (car items))
          (map proc (cdr items))))))
  (define separator "=========")

  (cons 'progn
    (map (lambda (x)
          (if (string? x)
            (syntax-list 'print x)
            (syntax-list 'print (to-string x) " = " (syntax-list 'to-string x))))
      (cons "===dprint===" exprs))))

(define (null? x) (nil? x))

(define (not x)
  (cond
    (x false)
    (true true)))

(define (append list1 list2)
  (if
    (null? list1)
    list2
    (cons (car list1) (append (cdr list1) list2))))

(define nil '())
(define (null? x) (nil? x))

(define (map proc items)
  (cond
    ((null? items) nil)
    (else
      (cons (proc (car items))
        (map proc (cdr items))))))
(define (accumulate op initial sequence)
  (if (null? sequence)
    initial
    (op (car sequence)
      (accumulate op initial (cdr sequence)))))
(define (map-n op . seqs)
  (define (c-args seqs)
    (cond
      ((null? (car seqs)) '())
      ((pair? seqs) (append
                     (list (map car seqs))
                     (c-args
                       (map cdr seqs))))))
  (accumulate
    (lambda (args acc)
      (cons (eval (cons op args)) acc))
    '()
    (c-args seqs)))

(define (newline) (print ""))

(define (filter predicate sequence)
  (cond
    ((null? sequence)
      nil)
    ((predicate (car sequence))
      (cons (car sequence) (filter predicate (cdr sequence))))
    (true
      (filter predicate (cdr sequence)))))

(defmacro (assert a b)
  (syntax-list 'if (syntax-list '= a b) '()
    (syntax-list 'print "assertion failed, found: "
      (syntax-list 'to-string a)
      " but expected: "
      (syntax-list 'to-string b)
      ". "
      (to-string a)
      " != "
      (to-string b))))

(define (reverse x)
  (def reverse-iter
    (lambda (x acc)
      (cond
        ((null? x) acc)
        (true (reverse-iter (cdr x) (cons (car x) acc))))))
  (reverse-iter x '()))

(define (accumulate op initial sequence)
  (if (null? sequence)
    initial
    (op (car sequence)
      (accumulate op initial (cdr sequence)))))

(define (enumerate-interval low high)
  (if (> low high)
    nil
    (cons low (enumerate-interval (+ low 1) high))))

(define (enumerate-tree tree)
  (cond ((null? tree) nil)
    ((not (pair? tree)) (list tree))
    (else (append (enumerate-tree (car tree))
           (enumerate-tree (cdr tree))))))

(define (length sequence)
  (accumulate (lambda (skip x) (+ 1 x)) 0 sequence))

(define (map proc items)
  (if
    (null? items)
    nil
    (cons (proc (car items))
      (map proc (cdr items)))))
(define (accumulate op initial sequence)
  (if (null? sequence)
    initial
    (op (car sequence)
      (accumulate op initial (cdr sequence)))))
(define (map-n op . seqs)
  (define (c-args seqs)
    (cond
      ((null? (car seqs)) '())
      ((pair? seqs) (append
                     (list (map car seqs))
                     (c-args
                       (map cdr seqs))))))
  (accumulate
    (lambda (args acc)
      (cons (eval (cons op args)) acc))
    '()
    (c-args seqs)))

(define (accumulate-n op init seqs)
  (if (null? (car seqs))
    nil
    (cons
      (accumulate op init (map car seqs))
      (accumulate-n op init (map cdr seqs)))))

(define (fold-left op initial sequence)
  (define (iter result rest)
    (if (null? rest)
      result
      (iter (op result (car rest))
        (cdr rest))))
  (iter initial sequence))

(define (fold-right op initial sequence)
  (if
    (null? sequence)
    initial
    (op
      (car sequence)
      (fold-right op initial (cdr sequence)))))
(define
  (flatmap proc seq)
  (accumulate append '() (map proc seq)))

(define (memq item x)
  (cond
    ((null? x) false)
    ((= item (car x)) x)
    (else (memq item (cdr x)))))

(assert (memq 'apple '(pear banana prune)) false)
(assert
  (memq 'apple '(x (apple sauce) y apple pear))
  '(apple pear))
(define (cadr list) (car (cdr list)))
(define (caddr list) (car (cdr (cdr list))))

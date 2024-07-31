(assert (list 'a 'b 'c) '(a b c))
(assert (list (list 'george)) '((george)))
(assert (cdr '(
               (x1 x2)
               (y1 y2)))
  '((y1 y2)))
(define (cadr l) (car (cdr l)))
(assert
  (cadr '((x1 x2) (y1 y2)))
  '(y1 y2))

(assert
  (pair? (car '(a short list)))
  false)

(assert
  (memq 'red '((red shoes) (blue socks)))
  false)

(assert
  (memq 'red '(red shoes blue socks))
  '(red shoes blue socks))

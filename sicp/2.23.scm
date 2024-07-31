(define (for-each f xs)
  (map f xs)
  true)
(for-each print (list 57 321 88))

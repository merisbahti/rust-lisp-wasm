(define (for-each f xs)
  (map f xs)
  true)
(for-each display (list 57 321 88))

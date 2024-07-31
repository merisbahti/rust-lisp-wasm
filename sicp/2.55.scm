; It's true because the expression  (car ''abracadabra)
; is just "syntactic sugar" for (car (quote (quote abracadabra)))
; which evaluates to `quote`

(assert (car ''abracadabra) 'quote)

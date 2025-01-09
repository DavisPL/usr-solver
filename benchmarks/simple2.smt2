; TEST CASE 2:
(declare-const x String)
(assert (and (str.in_re x (str.to_re "a")) (str.in_re x (str.to_re "b"))))
(check-sat)

; EXPECTED OUTPUT:
; one of
; - (x \cap a) \cap (x \cap b)
; - (x \cap a \cap b)
; - (x \cap a) \cdot (x \cap b)

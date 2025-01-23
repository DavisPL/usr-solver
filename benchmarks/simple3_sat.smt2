; TEST CASE 3:
(declare-const x String)
(declare-const y String)
(assert (and (str.in_re x (str.to_re "a")) (str.in_re y (str.to_re "b"))))
(check-sat)

; EXPECTED OUTPUT:
; one of:
; - (x \cap a) \cdot (y \cap b)
; - SAT(x \cap a) and SAT(y \cap b) <-- correct in this case

; Solution sketch
; (str.in_re x R) --> create a regex x \cap R
; (x1 \cap R1) and (x2 \cap R2) --> check if x1 == x2, if so rewrite as (x1 \cap R1 \cap R2), otherwise create (x1 \cap R1) \cdot (x2 \cap R2).
; optionally rewrite (x \cap R) as R if x does not occur anywhere else.

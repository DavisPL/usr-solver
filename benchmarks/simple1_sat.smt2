; TEST CASE 1:
(declare-const x String)
(assert (str.in_re x (re.++ (str.to_re "a") (str.to_re "b"))))
(check-sat)

; EXPECTED OUTPUT:
; one of:
; - a b
; - x \cap (a b)

; A generalization of this:
; let R be a USR and let w be a fresh string variable
; consider the USRs (i) R (ii) w \cap R
; Q: do SATISFIABLE(R) and SATISFIABLE(w \cap R) run in the same amount of time?

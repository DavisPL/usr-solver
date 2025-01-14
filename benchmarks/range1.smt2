; RANGE TEST:
(declare-const x String)
(assert (str.in_re x (re.range "0" "9")))
(check-sat)

; EXPECTED OUTPUT:
; x intersect (0 U 1 U 2 U 3 U 4 U 5 U 6 U 7 U 8 U 9)
; TEST CASE 1
; (Same as simple1_sat.smt2)

(declare-const x String)
(assert
    (let ((a1 (str.to_re "a")))
        (let ((a2 (str.to_re "b")))
            (str.in_re x (re.++ a1 a2))
        )
    )
)
(check-sat)

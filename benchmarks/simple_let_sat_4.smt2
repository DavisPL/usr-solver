; TEST CASE 4
; Same as the first 3, with multiple lets now

(declare-const x String)
(assert
    (let (
        (a1 (str.to_re "a"))
        (a2 (str.to_re "b"))
    )
    (str.in_re x (re.++ a1 a2)))
)
(check-sat)

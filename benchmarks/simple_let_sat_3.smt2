; TEST CASE 3
; Same as the first two but this one uses a let expression returning a formula, not a regex.

(declare-const x String)
(assert
    (let ((a1 (str.in_re x (re.++ (str.to_re "a") (str.to_re "b")))))
        a1
    )
)
(check-sat)

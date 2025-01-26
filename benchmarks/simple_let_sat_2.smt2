; TEST CASE 2
; Same as the first one but this one uses a let expression in the regex, not in the assert

(declare-const x String)
(assert (str.in_re x
    (let ((a1 (str.to_re "a")))
        (let ((a2 (str.to_re "b")))
            (re.++ a1 a2)
        )
    )
))
(check-sat)

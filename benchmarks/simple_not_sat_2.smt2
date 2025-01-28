(declare-const x String)
(assert
    (let ((a1 (str.in_re x (re.++ (str.to_re "a") (str.to_re "b")))))
        (not a1)
    )
)
(check-sat)
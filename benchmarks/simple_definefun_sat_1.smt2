(define-fun Witness () String (str.++ "a" "b"))
(assert (str.in_re Witness (re.++ (str.to_re "a") (str.to_re "b"))))
(check-sat)
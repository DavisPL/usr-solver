(declare-fun w () String)
(assert
 (str.in_re w
  (re.diff (re.diff (re.++ (re.* (str.to_re "b")) (str.to_re "a"))
   (str.to_re w))
   (re.++ (str.to_re "a") (str.to_re w)))))
(check-sat)

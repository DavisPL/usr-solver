; from GitHub issue:
(declare-fun a () String)
(declare-const x String)
(assert
 (str.in_re x
 (re.inter
   (re.++ (str.to_re a)(re.++ (str.to_re a) (str.to_re "bat")))
   (re.++ (str.to_re a) (re.++ (str.to_re a) (str.to_re a)))
	))
 )
(check-sat)

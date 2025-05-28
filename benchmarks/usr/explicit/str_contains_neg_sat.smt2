; Translated
(set-logic QF_S)


(declare-const s1 String)

(assert
	(not (str.contains s1 "abd"))
)
(assert
	(str.in_re s1
		
	(re.++
		(str.to_re "ab")
		(re.* re.allchar)
		(str.to_re "d")
	)
	)
)
(check-sat)

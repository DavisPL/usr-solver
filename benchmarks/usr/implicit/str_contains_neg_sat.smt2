; Translated
(set-logic QF_S)


(declare-const s1 String)
(declare-const x String)

(assert
 (str.in_re x
 (re.inter
	(str.to_re s1)
	(re.comp (re.++
		(re.* re.allchar)
		(str.to_re "abd")
		(re.* re.allchar)
	))
	(re.++
		(str.to_re "ab")
		(re.* re.allchar)
		(str.to_re "d")
	)
 )
)
 )
(check-sat)

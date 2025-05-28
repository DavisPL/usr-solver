; Translated
(set-logic QF_S)

(declare-const s1 String)

(assert (str.in_re s1 (re.++ (str.to_re "ab") (re.* re.allchar) (str.to_re "d"))))

(assert (str.in_re s1
	 (re.++
		
			re.allchar
			re.allchar
			re.allchar
			re.allchar
			re.allchar
			(re.* re.allchar)
	 )

)
)

(check-sat)


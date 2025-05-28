; Translated
(set-logic QF_S)

(declare-const s1 String)
(declare-const x String)

(assert
 (str.in_re x
 (re.inter
 (str.to_re s1)
 (re.++
	(re.* re.allchar)
	(str.to_re "pine")
	(re.* re.allchar)
 )
)
)
)
(check-sat)



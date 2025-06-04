(set-logic QF_S)

(declare-const s1 String)

(assert
	(str.contains s1 "pine")
)
(check-sat)



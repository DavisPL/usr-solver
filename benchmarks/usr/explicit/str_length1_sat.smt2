(set-logic QF_S)

(declare-const s1 String)

(assert (str.in_re s1 (re.++ (str.to_re "ab") re.allchar (str.to_re "d"))))

(assert (< (str.len s1) 5))

(check-sat)



(declare-const s1 String)
(declare-const y String)

;(assert (= "oz72" (str.substr s1 14 2)))
;(assert (= "mmlz" (str.substr s1 6 2)))
;(assert (= "0792" (str.substr s1 7 2)))

(assert
    (str.in_re y
        (re.++
            ; CONSTRAINT 1
            (re.union
                ; CASE 1.1
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "oz") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "oz")
                        (re.++ re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                ; CASE 1.2
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "oz"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                ; CASE 1.3
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "oz")
                        (str.to_re "")
                    )
                )
            )
            ; CONSTRAINT 2
            (re.union
                ; CASE 2.1
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "mmlz") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "mm")
                        (re.++ re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                ; CASE 2.2
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "mm"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                ; CASE 2.3
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "mmlz")
                        (str.to_re "")
                    )
                )
            )
            ; CONSTRAINT 3
            (re.union
                ; CASE 3.1
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "0792") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "0792")
                        (re.++ re.allchar re.allchar re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                ; CASE 3.2
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "0792"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                ; CASE 3.3
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "0792")
                        (str.to_re "")
                    )
                )
            )
        )
    )
)
(check-sat)

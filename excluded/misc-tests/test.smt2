(declare-const s1 String)
(declare-const y String)

;(assert (= "c" (str.substr s1 2 1)))
;(assert (= "aa" (str.substr s1 0 2)))
;(assert (= "b" (str.substr s1 1 1)))

(assert
    (str.in_re y
        (re.++
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar (str.to_re "c") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "c")
                        (re.++ re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar (str.to_re "c"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "c")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar (str.to_re "aa") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "aa")
                        (re.++ re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar (str.to_re "aa"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar (re.comp (re.++ re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "aa")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar (str.to_re "b") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "b")
                        (re.++ re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar (str.to_re "b"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar (re.comp (re.++ re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "b")
                        (str.to_re "")
                    )
                )
            )
        )
    )
)
(check-sat)

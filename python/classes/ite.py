class IfThenElse():
    def __init__(self, predicate, trueExpr, falseExpr):
        super().__init__()
        self.predicate = predicate
        self.trueExpr = trueExpr
        self.falseExpr = falseExpr
    '''

    def evaluate(self, valuation):
        if self.predicate.evaluate(valuation):
            return self.trueExpr.evaluate(valuation)
        return self.falseExpr.evaluate(valuation)
    
    '''

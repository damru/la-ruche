-- EP-004-S09: Plan comptable copropriété (arrêté 14/03/2005)
--
-- Read-only reference table seeded with common codes.
-- budget_line.account_code is optional (required only for legal document generation).

CREATE TABLE accounting_code (
    code     TEXT PRIMARY KEY,
    label    TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'general'
);

INSERT INTO accounting_code (code, label, category) VALUES
('602',   'Achats de fournitures non stockées',                   'charges'),
('6152',  'Entretien et réparations des biens immobiliers',       'charges'),
('6156',  'Entretien et réparations des biens mobiliers',         'charges'),
('6161',  'Primes d''assurance — multirisque immeuble',           'assurance'),
('6163',  'Primes d''assurance — responsabilité civile',          'assurance'),
('6226',  'Honoraires syndic',                                    'honoraires'),
('6228',  'Rémunérations d''intermédiaires divers',               'honoraires'),
('6231',  'Annonces et insertions légales',                       'frais'),
('6251',  'Déplacements et réceptions',                           'frais'),
('6270',  'Services bancaires',                                   'frais'),
('6282',  'Gardiennage et surveillance',                          'charges'),
('6311',  'Taxe foncière',                                        'taxes'),
('6412',  'Salaires et traitements',                              'personnel'),
('6450',  'Charges patronales',                                   'personnel'),
('7320',  'Provisions pour charges — copropriétaires',            'produits'),
('7321',  'Provisions pour travaux — copropriétaires',            'produits'),
('10200', 'Provisions pour charges de copropriété',               'fonds'),
('10300', 'Fonds de travaux (art. 14-2-1 loi 1965)',              'fonds');

ALTER TABLE budget_line
    ADD COLUMN account_code TEXT REFERENCES accounting_code(code);

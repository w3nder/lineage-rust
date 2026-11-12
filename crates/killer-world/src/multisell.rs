use l2_net::game::{MultiSellIngredient, MultiSellList, MultiSellProduct};

pub const PRODUCT_SLOTS: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSellOffer {
    pub entry_id: i32,
    pub stackable: u8,
    pub products: [Option<MultiSellProduct>; PRODUCT_SLOTS],
    pub product_count: usize,
    pub ingredients: Vec<MultiSellIngredient>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultiSellState {
    pub list_id: i32,
    pub offers: Vec<MultiSellOffer>,
    pub finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductSlotsExceeded {
    pub entry_id: i32,
}

impl MultiSellState {
    pub fn receive(&mut self, page: &MultiSellList) -> Result<(), ProductSlotsExceeded> {
        if page.page == 1 {
            self.offers.clear();
            self.list_id = page.list_id;
        }
        self.finished = false;
        for entry in &page.entries {
            for (index, product) in entry.products.iter().enumerate() {
                let existing = self.offers.iter().position(|offer| offer.entry_id == entry.entry_id);
                if index >= PRODUCT_SLOTS || existing.is_some_and(|at| self.offers[at].product_count >= PRODUCT_SLOTS) {
                    return Err(ProductSlotsExceeded { entry_id: entry.entry_id });
                }
                let at = match existing {
                    Some(at) => {
                        self.offers[at].product_count += 1;
                        at
                    }
                    None => {
                        self.offers.push(MultiSellOffer {
                            entry_id: entry.entry_id, stackable: entry.stackable,
                            products: [None; PRODUCT_SLOTS], product_count: 1, ingredients: Vec::new(),
                        });
                        self.offers.len() - 1
                    }
                };
                self.offers[at].stackable = entry.stackable;
                self.offers[at].products[index] = Some(*product);
            }
            if let Some(offer) = self.offers.iter_mut().find(|offer| offer.entry_id == entry.entry_id) {
                offer.ingredients.extend_from_slice(&entry.ingredients);
            }
        }
        self.finished = page.finished;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::game::MultiSellEntry;

    fn packet(page: i32, entry_id: i32) -> MultiSellList {
        MultiSellList {
            list_id: 10, page, finished: false, page_size: 40,
            entries: vec![MultiSellEntry {
                entry_id, stackable: 1,
                products: vec![MultiSellProduct { item_id: 1, body_part: 0x80, type2: 0, count: 1, enchant: 0 }],
                ingredients: vec![MultiSellIngredient { item_id: 57, type2: 4, count: 100, enchant: 0 }],
            }],
        }
    }

    #[test]
    fn begin_list_100a41c0_only_resets_on_page_one() {
        let mut state = MultiSellState::default();
        state.receive(&packet(0, 1)).unwrap();
        assert_eq!(state.list_id, 0);
        state.receive(&packet(1, 2)).unwrap();
        assert_eq!(state.offers.len(), 1);
        assert_eq!(state.offers[0].entry_id, 2);
        let mut next = packet(2, 3);
        next.list_id = 999;
        next.page_size = -1;
        next.finished = true;
        state.receive(&next).unwrap();
        assert_eq!(state.list_id, 10);
        assert_eq!(state.offers.len(), 2);
        assert!(state.finished);
        state.receive(&packet(1, 4)).unwrap();
        assert!(!state.finished);
        assert_eq!(state.offers.len(), 1);
    }

    #[test]
    fn entry_100a33b0_overwrites_slots_counts_calls_and_appends_ingredients() {
        let mut state = MultiSellState::default();
        state.receive(&packet(1, 1)).unwrap();
        let mut next = packet(2, 1);
        next.entries[0].products[0].item_id = 2;
        next.entries[0].stackable = 0;
        state.receive(&next).unwrap();
        let offer = &state.offers[0];
        assert_eq!(offer.product_count, 2);
        assert_eq!(offer.products[0].unwrap().item_id, 2);
        assert!(offer.products[1].is_none());
        assert_eq!(offer.stackable, 0);
        assert_eq!(offer.ingredients.len(), 2);
        next.entries[0].products.clear();
        state.receive(&next).unwrap();
        assert_eq!(state.offers[0].ingredients.len(), 3);
        next.entries[0].entry_id = 9;
        state.receive(&next).unwrap();
        assert_eq!(state.offers.len(), 1);
    }

    #[test]
    fn offer_ctor_100a2ee0_bounds_products_but_not_the_number_of_offers() {
        let mut state = MultiSellState::default();
        for id in 0..101 { state.receive(&packet(2, id)).unwrap(); }
        assert_eq!(state.offers.len(), 101);
        let mut page = packet(1, 1);
        page.entries[0].products = vec![page.entries[0].products[0]; PRODUCT_SLOTS];
        state.receive(&page).unwrap();
        assert_eq!(state.offers[0].product_count, PRODUCT_SLOTS);
        page.page = 2;
        assert_eq!(state.receive(&page), Err(ProductSlotsExceeded { entry_id: 1 }));
        assert_eq!(state.offers[0].product_count, PRODUCT_SLOTS);
    }
}
